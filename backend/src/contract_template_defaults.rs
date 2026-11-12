use serde_json::{Value, json};

pub(crate) struct DefaultContractTemplate {
    pub key: &'static str,
    pub name: &'static str,
    pub legacy_name: &'static str,
    pub lease_type: &'static str,
    pub locale: &'static str,
    pub is_active: bool,
    pub content: &'static str,
}

pub(crate) const DEFAULT_CONTRACT_TEMPLATES: [DefaultContractTemplate; 8] = [
    DefaultContractTemplate {
        key: "residential-lease-ar",
        name: "نموذج عقد إيجار سكني",
        legacy_name: "قالب إيجار سكني (فارغ)",
        lease_type: "residential",
        locale: "ar",
        is_active: false,
        content: r#"# عقد إيجار سكني

رقم العقد: {{contract.number}}  
تاريخ العقد: {{contract.date}}

## أطراف العقد
- المؤجر / ممثل العقار: {{organization.name}}
- المستأجر: {{tenant.full_name}}
- البريد الإلكتروني: {{tenant.email}}
- رقم التواصل: {{tenant.phone}}
- عنوان المستأجر: {{tenant.address}}

## العين المؤجرة
- العقار: {{property.name}} ({{property.code}})
- العنوان: {{property.address}}، {{property.city}}
- الوحدة: {{unit.number}} — {{unit.type}}
- الطابق: {{unit.floor}} · المساحة: {{unit.area}}
- غرف النوم: {{unit.bedrooms}} · دورات المياه: {{unit.bathrooms}}

## مدة الإيجار
تبدأ مدة الإيجار في {{contract.start_date}} وتنتهي في {{contract.end_date}}.

## الأجرة والمبالغ
- قيمة الإيجار المتفق عليها: {{unit.rent}} ({{unit.currency}})
- مبلغ التأمين المتفق عليه: {{contract.deposit}}
- مواعيد السداد وطريقته: [تُستكمل بالاتفاق بين الطرفين]

## تفاصيل وشروط الاتفاق
[يُضاف هنا ما اتفق عليه الطرفان بشأن التسليم والصيانة والمرافق وأي شروط خاصة، ثم يُراجع قبل الاعتماد.]

## إقرار وتوقيع الطرفين
يقر الطرفان بمراجعة البيانات والشروط المدونة أعلاه، وتُستكمل التوقيعات بعد المراجعة والاعتماد.

المؤجر / الممثل: ____________________    التاريخ: __________

المستأجر: __________________________    التاريخ: __________
"#,
    },
    DefaultContractTemplate {
        key: "commercial-lease-ar",
        name: "نموذج عقد إيجار تجاري",
        legacy_name: "قالب إيجار تجاري (فارغ)",
        lease_type: "commercial",
        locale: "ar",
        is_active: false,
        content: r#"# عقد إيجار تجاري

رقم العقد: {{contract.number}}  
تاريخ العقد: {{contract.date}}

## أطراف العقد
- المؤجر / ممثل العقار: {{organization.name}}
- المستأجر / المنشأة: {{tenant.full_name}}
- البريد الإلكتروني: {{tenant.email}}
- رقم التواصل: {{tenant.phone}}
- العنوان: {{tenant.address}}

## الوحدة والغرض من الاستخدام
- العقار: {{property.name}} ({{property.code}})
- العنوان: {{property.address}}، {{property.city}}
- الوحدة التجارية: {{unit.number}} — {{unit.type}}
- الطابق: {{unit.floor}} · المساحة: {{unit.area}}
- الغرض من الاستخدام: [يُحدد النشاط المتفق عليه]

## مدة الإيجار
تبدأ مدة الإيجار في {{contract.start_date}} وتنتهي في {{contract.end_date}}.

## الأجرة والمبالغ
- قيمة الإيجار المتفق عليها: {{unit.rent}} ({{unit.currency}})
- مبلغ التأمين المتفق عليه: {{contract.deposit}}
- جدول السداد والرسوم إن وجدت: [تُستكمل بالاتفاق بين الطرفين]

## شروط النشاط والتسليم
[تُستكمل تفاصيل التسليم والتجهيزات والمرافق وشروط النشاط وأي التزامات خاصة، وتُراجع قبل الاعتماد.]

## إقرار وتوقيع الطرفين
يقر الطرفان بمراجعة بيانات الوحدة والغرض من استخدامها والشروط المدونة أعلاه.

المؤجر / الممثل: ____________________    التاريخ: __________

المستأجر / المفوض: __________________    الصفة: ___________

التوقيع: ___________________________    التاريخ: __________
"#,
    },
    DefaultContractTemplate {
        key: "short-term-lease-ar",
        name: "نموذج عقد إيجار قصير الأجل",
        legacy_name: "قالب إيجار قصير الأجل (فارغ)",
        lease_type: "short_term",
        locale: "ar",
        is_active: false,
        content: r#"# اتفاق إيجار قصير الأجل

رقم الاتفاق: {{contract.number}}  
تاريخ الاتفاق: {{contract.date}}

## بيانات الطرفين
- المؤجر / ممثل العقار: {{organization.name}}
- المستأجر: {{tenant.full_name}}
- البريد الإلكتروني: {{tenant.email}}
- رقم التواصل: {{tenant.phone}}
- عنوان المستأجر: {{tenant.address}}

## مكان الإقامة
- العقار: {{property.name}} ({{property.code}})
- العنوان: {{property.address}}، {{property.city}}
- الوحدة: {{unit.number}} — {{unit.type}}
- الطابق: {{unit.floor}} · المساحة: {{unit.area}}

## فترة الإقامة
من {{contract.start_date}} إلى {{contract.end_date}}.

## المقابل المالي
- قيمة الإيجار المتفق عليها: {{unit.rent}} ({{unit.currency}})
- مبلغ التأمين إن وجد: {{contract.deposit}}
- مواعيد وطريقة السداد: [تُستكمل بالاتفاق بين الطرفين]

## ترتيبات الإقامة والتسليم
[تُستكمل أوقات الدخول والمغادرة، وحالة الوحدة ومحتوياتها، وأي ترتيبات أو شروط خاصة متفق عليها.]

## إقرار وتوقيع الطرفين
يراجع الطرفان بيانات الإقامة والمبالغ والترتيبات قبل التوقيع.

المؤجر / الممثل: ____________________    التاريخ: __________

المستأجر: __________________________    التاريخ: __________
"#,
    },
    DefaultContractTemplate {
        key: "residential-lease-en",
        name: "Residential Lease Agreement",
        legacy_name: "Residential lease (blank)",
        lease_type: "residential",
        locale: "en",
        is_active: false,
        content: r#"# Residential Lease Agreement

Agreement number: {{contract.number}}  
Agreement date: {{contract.date}}

## Parties
- Landlord / property representative: {{organization.name}}
- Tenant: {{tenant.full_name}}
- Email: {{tenant.email}}
- Phone: {{tenant.phone}}
- Tenant address: {{tenant.address}}

## Premises
- Property: {{property.name}} ({{property.code}})
- Address: {{property.address}}, {{property.city}}
- Unit: {{unit.number}} — {{unit.type}}
- Floor: {{unit.floor}} · Area: {{unit.area}}
- Bedrooms: {{unit.bedrooms}} · Bathrooms: {{unit.bathrooms}}

## Lease term
The agreed lease term begins on {{contract.start_date}} and ends on {{contract.end_date}}.

## Rent and amounts
- Agreed rent: {{unit.rent}} ({{unit.currency}})
- Agreed security deposit: {{contract.deposit}}
- Payment schedule and method: [Complete as agreed by the parties]

## Agreed terms
[Add the parties' agreed handover, maintenance, utilities, and any special terms. Review before approval.]

## Acknowledgment and signatures
The parties confirm that they have reviewed the details and terms above. Complete signatures after review and approval.

Landlord / representative: ______________    Date: __________

Tenant: ________________________________    Date: __________
"#,
    },
    DefaultContractTemplate {
        key: "commercial-lease-en",
        name: "Commercial Lease Agreement",
        legacy_name: "Commercial lease (blank)",
        lease_type: "commercial",
        locale: "en",
        is_active: false,
        content: r#"# Commercial Lease Agreement

Agreement number: {{contract.number}}  
Agreement date: {{contract.date}}

## Parties
- Landlord / property representative: {{organization.name}}
- Tenant / business: {{tenant.full_name}}
- Email: {{tenant.email}}
- Phone: {{tenant.phone}}
- Address: {{tenant.address}}

## Premises and permitted use
- Property: {{property.name}} ({{property.code}})
- Address: {{property.address}}, {{property.city}}
- Commercial unit: {{unit.number}} — {{unit.type}}
- Floor: {{unit.floor}} · Area: {{unit.area}}
- Agreed business use: [Specify the parties' agreed activity]

## Lease term
The agreed lease term begins on {{contract.start_date}} and ends on {{contract.end_date}}.

## Rent and amounts
- Agreed rent: {{unit.rent}} ({{unit.currency}})
- Agreed security deposit: {{contract.deposit}}
- Payment schedule and any fees: [Complete as agreed by the parties]

## Handover and business terms
[Complete handover, fit-out, utilities, permitted-use details, and any special obligations. Review before approval.]

## Acknowledgment and signatures
The parties confirm that they have reviewed the premises, permitted use, and terms above.

Landlord / representative: ______________    Date: __________

Tenant / authorized signatory: __________    Title: __________

Signature: _____________________________    Date: __________
"#,
    },
    DefaultContractTemplate {
        key: "short-term-lease-en",
        name: "Short-Term Stay Agreement",
        legacy_name: "Short-term lease (blank)",
        lease_type: "short_term",
        locale: "en",
        is_active: false,
        content: r#"# Short-Term Stay Agreement

Agreement number: {{contract.number}}  
Agreement date: {{contract.date}}

## Parties
- Host / property representative: {{organization.name}}
- Guest: {{tenant.full_name}}
- Email: {{tenant.email}}
- Phone: {{tenant.phone}}
- Guest address: {{tenant.address}}

## Accommodation
- Property: {{property.name}} ({{property.code}})
- Address: {{property.address}}, {{property.city}}
- Unit: {{unit.number}} — {{unit.type}}
- Floor: {{unit.floor}} · Area: {{unit.area}}

## Stay period
The agreed stay begins on {{contract.start_date}} and ends on {{contract.end_date}}.

## Price and amounts
- Agreed rent: {{unit.rent}} ({{unit.currency}})
- Security deposit, if applicable: {{contract.deposit}}
- Payment schedule and method: [Complete as agreed by the parties]

## Handover and stay arrangements
[Complete check-in/check-out times, unit condition and inventory, and any agreed stay arrangements.]

## Acknowledgment and signatures
The parties should review the stay details, amounts, and arrangements before signing.

Host / representative: _________________    Date: __________

Guest: ________________________________    Date: __________
"#,
    },
    DefaultContractTemplate {
        key: "payment-receipt-ar",
        name: "قالب إيصال قبض",
        legacy_name: "",
        lease_type: "receipt",
        locale: "ar",
        is_active: true,
        content: r#"# إيصال قبض

## بيانات الإيصال
- رقم الإيصال: {{payment.receipt_number}}
- تاريخ القبض: {{payment.date}}
- الحالة: {{payment.status}}

## استلمنا من
- المستأجر: {{payment.tenant}}

## تفاصيل المبلغ
- المبلغ: {{payment.amount}} ({{payment.currency}})
- توزيع الدفعة: {{payment.allocations}}
- طريقة الدفع: {{payment.method}}
- المرجع: {{payment.reference}}

## ملاحظات
{{payment.notes}}

تم إصدار هذا الإيصال بواسطة {{organization.name}}.
"#,
    },
    DefaultContractTemplate {
        key: "payment-receipt-en",
        name: "Payment Receipt Template",
        legacy_name: "",
        lease_type: "receipt",
        locale: "en",
        is_active: true,
        content: r#"# Payment Receipt

## Receipt details
- Receipt number: {{payment.receipt_number}}
- Date received: {{payment.date}}
- Status: {{payment.status}}

## Received from
- Tenant: {{payment.tenant}}

## Payment details
- Amount: {{payment.amount}} ({{payment.currency}})
- Allocation: {{payment.allocations}}
- Payment method: {{payment.method}}
- Reference: {{payment.reference}}

## Notes
{{payment.notes}}

Issued by {{organization.name}}.
"#,
    },
];

pub(crate) fn default_template_placeholders(template: &DefaultContractTemplate) -> String {
    let is_receipt = template.lease_type == "receipt";
    let is_arabic = template.locale == "ar";
    json!({
        "design": {
            "paper_size": if is_receipt { "a5" } else { "a4" },
            "accent_color": "#C7962F",
            "font_family": if is_arabic { "Tahoma" } else { "Arial" },
            "font_size": if is_receipt { 14 } else { 16 },
            "alignment": "start",
            "page_margin_mm": if is_receipt { 12 } else { 18 },
            "show_branding": true,
            "brand_label": "",
            "show_contract_number": true,
            "show_footer": true,
            "footer_text": if is_receipt {
                if is_arabic { "إيصال صادر عن النظام." } else { "System-generated receipt." }
            } else if is_arabic {
                "مسودة قابلة للتخصيص، وتخضع للقواعد الآمرة في الجمهورية اليمنية."
            } else {
                "Editable draft; mandatory laws of the Republic of Yemen apply."
            },
            "organization_logo_data_url": null
        },
        "docforge_document": default_docforge_document(template)
    })
    .to_string()
}

pub(crate) fn default_template_content(template: &DefaultContractTemplate) -> String {
    if template.lease_type == "receipt" {
        return if template.locale == "ar" {
            r#"# إيصال قبض

رقم الإيصال: {{payment.receipt_number}}
التاريخ: {{payment.date}}
الحالة: {{payment.status}}

## استلمنا من
{{payment.tenant}}

## المبلغ المستلم
{{payment.amount}}

## توزيع الدفعة
{{payment.allocations}}

طريقة الدفع: {{payment.method}}
المرجع: {{payment.reference}}

ملاحظات: {{payment.notes}}
"#
            .to_string()
        } else {
            r#"# Payment Receipt

Receipt no.: {{payment.receipt_number}}
Date: {{payment.date}}
Status: {{payment.status}}

## Received from
{{payment.tenant}}

## Amount received
{{payment.amount}}

## Payment allocation
{{payment.allocations}}

Method: {{payment.method}}
Reference: {{payment.reference}}

Notes: {{payment.notes}}
"#
            .to_string()
        };
    }

    let is_arabic = template.locale == "ar";
    let title = match (template.lease_type, is_arabic) {
        ("commercial", true) => "عقد إيجار تجاري",
        ("short_term", true) => "اتفاق إيجار قصير الأجل",
        ("commercial", false) => "Commercial Lease Agreement",
        ("short_term", false) => "Short-Term Stay Agreement",
        (_, true) => "عقد إيجار سكني",
        (_, false) => "Residential Lease Agreement",
    };
    let mut content = format!("# {title}\n\n");

    content.push_str(if is_arabic {
        r#"رقم العقد: {{contract.number}}
تاريخ العقد: {{contract.date}}

## أولاً: أطراف العقد
1. المؤجر / الطرف الأول: {{organization.name}}، ويشار إليه في هذا العقد بـ«المؤجر».
2. المستأجر / الطرف الثاني: {{tenant.full_name}}، رقم الهوية: [يُستكمل]، الهاتف: {{tenant.phone}}، البريد: {{tenant.email}}، العنوان: {{tenant.address}}.

## ثانياً: العين المؤجرة
العقار: {{property.name}} (الرمز {{property.code}})، العنوان: {{property.address}}، {{property.city}}.
الوحدة: {{unit.number}}، النوع: {{unit.type}}، الطابق: {{unit.floor}}، المساحة: {{unit.area}}.
غرف النوم: {{unit.bedrooms}} · دورات المياه: {{unit.bathrooms}}.
"#
    } else {
        r#"Agreement no.: {{contract.number}}
Agreement date: {{contract.date}}

## 1. Parties
1. Landlord / First Party: {{organization.name}} (the “Landlord”).
2. Tenant / Second Party: {{tenant.full_name}}, ID/passport: [complete], phone: {{tenant.phone}}, email: {{tenant.email}}, address: {{tenant.address}}.

## 2. Leased Premises
Property: {{property.name}} (code {{property.code}}), address: {{property.address}}, {{property.city}}.
Unit: {{unit.number}}, type: {{unit.type}}, floor: {{unit.floor}}, area: {{unit.area}}.
Bedrooms: {{unit.bedrooms}} · Bathrooms: {{unit.bathrooms}}.
"#
    });

    content.push_str(if is_arabic {
        r#"
## ثالثاً: المدة والأجرة
مدة العقد من {{contract.start_date}} إلى {{contract.end_date}}. الأجرة المتفق عليها: {{unit.rent}} {{unit.currency}}، وتُدفع في المواعيد والطريقة الآتية: [يُحدد موعد السداد ودوريته وطريقته].
التأمين إن وجد: {{contract.deposit}}. ويُوثق استلام أي مبلغ بموجب إيصال، وتُحدد معالجة التأمين عند انتهاء العقد وفق الاتفاق المكتوب والقانون النافذ.

## رابعاً: محضر التسليم والبيانات الخاصة
تاريخ التسليم: [يُستكمل]. حالة الوحدة والعدادات والمفاتيح والمنقولات: [تُثبت في محضر جرد موقّع ويُرفق بالعقد].
شروط خاصة متفق عليها: [تُستكمل، أو يشار إلى الملحق المرفق].
"#
    } else {
        r#"
## 3. Term and Rent
The term runs from {{contract.start_date}} to {{contract.end_date}}. Agreed rent: {{unit.rent}} {{unit.currency}}, payable on the following dates and by the following method: [complete the due dates, frequency, and method].
Security deposit, if any: {{contract.deposit}}. Each payment must be documented by a receipt. The deposit treatment at the end of the term must be stated in writing and remain subject to applicable law.

## 4. Handover and Special Terms
Handover date: [complete]. Condition of the unit, meters, keys, and included items: [record in a signed inventory attached to this agreement].
Special agreed terms: [complete or identify the attached addendum].
"#
    });

    content.push_str("[[PAGE_BREAK]]\n\n");
    content.push_str(if is_arabic {
        r#"## الشروط والأحكام

### المادة (1): نطاق الاتفاق
يؤجر المؤجر للمستأجر العين المبيّنة في هذا العقد للغرض المحدد فيه، ويقر الطرفان بصحة البيانات التي قدماها. ويعد محضر التسليم والجرد والملحقات الموقعة جزءاً من الاتفاق.

### المادة (2): الاستعمال
"#
    } else {
        r#"## Terms and Conditions

### Article 1 — Scope
The Landlord lets the premises identified above to the Tenant for the stated purpose. Each party confirms the accuracy of the information it supplied. Any signed handover record, inventory, and addendum forms part of this agreement.

### Article 2 — Permitted Use
"#
    });
    content.push_str(match (template.lease_type, is_arabic) {
        ("commercial", true) => {
            "يقتصر الاستعمال على النشاط التجاري المحدد كتابةً في الملحق. ويتولى المستأجر استصدار التصاريح اللازمة لنشاطه، دون إخلال بالتزامات المؤجر أو الأحكام الآمرة."
        }
        ("short_term", true) => {
            "تحدد أوقات الدخول والمغادرة وعدد الشاغلين وأي خدمات مشمولة كتابةً في الملحق. ويلتزم المستأجر بالمحافظة على الوحدة ومحتوياتها وعدم تمكين الغير من الإقامة على خلاف الاتفاق."
        }
        (_, true) => {
            "تستعمل الوحدة للسكن، ولا يغير الغرض أو يُمكّن الغير من الانتفاع بها أو يؤجرها من الباطن إلا بموافقة كتابية مسبقة، مع مراعاة ما يجيزه القانون."
        }
        ("commercial", false) => {
            "Use is limited to the commercial activity specified in writing in an addendum. The Tenant is responsible for obtaining permits for its activity, without prejudice to the Landlord’s duties or mandatory law."
        }
        ("short_term", false) => {
            "Check-in and check-out times, permitted occupants, and included services must be stated in an addendum. The Tenant must care for the premises and included items and may not provide occupancy contrary to this agreement."
        }
        (_, false) => {
            "The unit is for residential use. The Tenant may not change its use, assign it, or sublet it without prior written consent, subject to any rights that cannot be waived under applicable law."
        }
    });
    content.push_str(if is_arabic {
        r#"

### المادة (3): الأجرة والتأمين
يلتزم المستأجر بسداد الأجرة في المواعيد والطريقة المبينة أعلاه. ولا يُفرض رسم أو غرامة غير منصوص عليها كتابةً أو غير جائزة قانوناً. ويُسوى التأمين عند انتهاء العلاقة بعد توثيق المستحقات والأضرار التي تثبت وفق القانون والاتفاق.

### المادة (4): الصيانة والمرافق
توزع مسؤوليات الصيانة والمرافق والرسوم في الملحق أو وفق القواعد القانونية النافذة. ويبلغ كل طرف الآخر دون تأخير عن العطل أو الضرر الجوهري. ولا يدخل المؤجر الوحدة إلا بموافقة المستأجر أو في حالة طارئة أو حيث يجيز القانون ذلك.

### المادة (5): التغييرات والتنازل
لا يجري المستأجر تغييراً جوهرياً أو إضافة ثابتة، ولا يتنازل عن العقد أو يؤجر من الباطن، إلا بموافقة كتابية مسبقة، وبما لا يخالف القانون. وتحدد كتابةً معالجة الإضافات عند انتهاء العقد.

### المادة (6): المدة والانتهاء
ينتهي العقد بانقضاء مدته ما لم يتفق الطرفان كتابةً على التجديد أو يترتب خلاف ذلك بموجب حكم آمر. ويكون الإنهاء والإشعار وتسليم الوحدة وفق ما اتفق عليه الطرفان وبما لا يخالف القانون النافذ.

### المادة (7): الإخلال وتسوية النزاع
يخطر الطرف المتضرر الطرف الآخر كتابةً بالمخالفة ويمنحه مهلة المعالجة المتفق عليها، إن وجدت، مع مراعاة الإجراءات والحقوق المقررة قانوناً. ويسعى الطرفان إلى التسوية الودية، وإلا يُعرض النزاع على الجهة القضائية اليمنية المختصة مكانياً ونوعياً وفق القانون.

### المادة (8): أحكام ختامية
لا يعدل هذا العقد إلا بمحرر مكتوب موقع من الطرفين. ويحرر من نسختين أصليتين، تسلم كل طرف نسخة، وتعد الملحقات الموقعة جزءاً منه. وإذا حرر بلغتين، يحدد الطرفان النسخة المعتمدة عند التوقيع.

## الإقرار والتوقيع
يقر الطرفان بقراءة البيانات والشروط وفهمها واستلام نسخة من العقد وملحقاته. هذه صيغة إرشادية تُستكمل بياناتها وتُراجع وفق الوقائع والقواعد الآمرة في الجمهورية اليمنية قبل التوقيع.

المؤجر / الطرف الأول: ____________________  التوقيع: ______________  التاريخ: __________
المستأجر / الطرف الثاني: __________________  التوقيع: ______________  التاريخ: __________
الشاهد الأول: ____________________________  الشاهد الثاني: ____________________________
"#
    } else {
        r#"

### Article 3 — Rent and Deposit
The Tenant must pay rent on the dates and by the method stated above. No fee or penalty applies unless agreed in writing and permitted by law. Any deposit must be reconciled at the end of the tenancy against documented amounts or damage, as permitted by the written agreement and applicable law.

### Article 4 — Maintenance and Utilities
Responsibility for maintenance, utilities, and charges must be stated in an addendum or determined by applicable law. Each party must promptly notify the other of material damage or defects. The Landlord may enter only with the Tenant’s consent, in an emergency, or as permitted by law.

### Article 5 — Alterations and Assignment
The Tenant may not make material alterations, install fixtures, assign this agreement, or sublet without prior written consent, subject to applicable law. The parties must state in writing how approved additions are treated at the end of the tenancy.

### Article 6 — Term and Ending the Agreement
The agreement ends at the stated expiry date unless the parties agree in writing to renew or a mandatory rule provides otherwise. Notice, termination, and handover must follow the parties’ written agreement and applicable law.

### Article 7 — Breach and Disputes
The affected party must give written notice of a breach and any agreed opportunity to remedy it, subject to procedures and rights provided by law. The parties will first seek an amicable resolution; unresolved disputes are submitted to the competent Yemeni court under applicable jurisdiction rules.

### Article 8 — General
Any amendment must be in writing and signed by both parties. This agreement is executed in two originals, one for each party; signed addenda form part of it. If bilingual versions are executed, the parties must identify the controlling version when signing.

## Acknowledgment and Signatures
Each party confirms that it has read and understood the terms and received a copy of this agreement and its attachments. This is an editable drafting template; complete and review it against the facts and mandatory laws of the Republic of Yemen before signing.

Landlord / First Party: ____________________  Signature: ____________  Date: __________
Tenant / Second Party: ____________________  Signature: ____________  Date: __________
Witness 1: _______________________________  Witness 2: ______________________________
"#
    });

    content
}

fn default_docforge_document(template: &DefaultContractTemplate) -> String {
    let is_receipt = template.lease_type == "receipt";
    let is_arabic = template.locale == "ar";
    let locale = if is_arabic { "ar" } else { "en" };
    let direction = if is_arabic { "rtl" } else { "ltr" };
    let width = if is_receipt { 148.0 } else { 210.0 };
    let inset = if is_receipt { 10.0 } else { 18.0 };
    let navy = "#17324D";
    let gold = "#C7962F";
    let ink = "#263445";
    let muted = "#65758B";
    let soft = "#F3F6FA";
    let paper_size = if is_receipt { "A5" } else { "A4" };
    let font_family = if is_arabic { "Tahoma" } else { "Arial" };
    let mut children = vec![
        shape_node(
            "brand-band",
            0.0,
            0.0,
            width,
            if is_receipt { 29.0 } else { 36.0 },
            navy,
            navy,
            0.0,
        ),
        image_node(
            "organization-logo",
            "{{organization.logo_url}}",
            inset,
            if is_receipt { 5.0 } else { 7.0 },
            if is_receipt { 17.0 } else { 22.0 },
            if is_receipt { 18.0 } else { 22.0 },
        ),
        text_node(
            "organization-name",
            "{{organization.name}}",
            if is_receipt { 32.0 } else { 46.0 },
            if is_receipt { 7.0 } else { 9.0 },
            width - if is_receipt { 42.0 } else { 64.0 },
            8.0,
            if is_receipt { 13.0 } else { 16.0 },
            true,
            "#FFFFFF",
            "start",
            direction,
            font_family,
        ),
        text_node(
            "brand-caption",
            if is_arabic {
                "إدارة العقارات"
            } else {
                "PROPERTY MANAGEMENT"
            },
            if is_receipt { 32.0 } else { 46.0 },
            if is_receipt { 17.0 } else { 21.0 },
            width - if is_receipt { 42.0 } else { 64.0 },
            5.0,
            8.0,
            false,
            "#E8C979",
            "start",
            direction,
            font_family,
        ),
        shape_node(
            "brand-rule",
            inset,
            if is_receipt { 29.0 } else { 36.0 },
            width - inset * 2.0,
            1.0,
            gold,
            gold,
            0.0,
        ),
    ];

    let mut pages = Vec::new();
    if is_receipt {
        add_receipt_layout(
            &mut children,
            width,
            inset,
            navy,
            gold,
            ink,
            muted,
            soft,
            locale,
            direction,
            font_family,
        );
        pages.push(json!({
            "type": "Page",
            "id": "page-1",
            "name": if is_arabic { "الصفحة الأولى" } else { "Page 1" },
            "children": children
        }));
    } else {
        add_lease_layout(
            &mut children,
            template,
            width,
            inset,
            navy,
            ink,
            muted,
            soft,
            locale,
            direction,
            font_family,
        );
        let mut page_two_children: Vec<Value> = children.iter().take(5).cloned().collect();
        for node in &mut page_two_children {
            let id = node["id"].as_str().unwrap_or("brand").to_string();
            node["id"] = json!(format!("page-2-{id}"));
            node["name"] = json!(format!("page-2-{id}"));
        }
        pages.push(json!({
            "type": "Page",
            "id": "page-1",
            "name": if is_arabic { "البيانات الأساسية" } else { "Parties and Terms" },
            "children": children
        }));
        add_lease_legal_page(
            &mut page_two_children,
            template,
            width,
            inset,
            navy,
            gold,
            ink,
            muted,
            locale,
            direction,
            font_family,
        );
        pages.push(json!({
            "type": "Page",
            "id": "page-2",
            "name": if is_arabic { "الشروط والتوقيع" } else { "Terms and Signatures" },
            "children": page_two_children
        }));
    }

    json!({
        "version": "2.0",
        "meta": {
            "name": template.name,
            "documentType": template.lease_type,
            "entityType": template.lease_type,
            "locale": locale,
            "pageSetup": {
                "size": paper_size,
                "orientation": "portrait",
                "margins": {
                    "top": inset,
                    "bottom": inset,
                    "left": inset,
                    "right": inset
                },
                "unit": "mm"
            }
        },
        "variables": {},
        "assets": {},
        "pages": pages
    })
    .to_string()
}

fn add_receipt_layout(
    children: &mut Vec<Value>,
    width: f64,
    inset: f64,
    navy: &str,
    gold: &str,
    ink: &str,
    muted: &str,
    soft: &str,
    locale: &str,
    direction: &str,
    font_family: &str,
) {
    let is_arabic = locale == "ar";
    let content_width = width - inset * 2.0;
    let title = if is_arabic {
        "إيصال قبض"
    } else {
        "PAYMENT RECEIPT"
    };
    push_text(
        children,
        "receipt-title",
        title,
        inset,
        35.0,
        content_width,
        10.0,
        19.0,
        true,
        navy,
        "center",
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "receipt-info-card",
        inset,
        48.0,
        content_width,
        23.0,
        soft,
        "#DFE5ED",
        0.4,
    ));
    push_text(
        children,
        "receipt-number",
        if is_arabic {
            "رقم الإيصال: {{payment.receipt_number}}"
        } else {
            "Receipt no.: {{payment.receipt_number}}"
        },
        inset + 4.0,
        51.0,
        content_width * 0.53,
        7.0,
        9.0,
        true,
        ink,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "receipt-date",
        if is_arabic {
            "التاريخ: {{payment.date}}"
        } else {
            "Date: {{payment.date}}"
        },
        inset + content_width * 0.56,
        51.0,
        content_width * 0.40,
        7.0,
        8.0,
        false,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "receipt-status",
        if is_arabic {
            "الحالة: {{payment.status}}"
        } else {
            "Status: {{payment.status}}"
        },
        inset + 4.0,
        60.0,
        content_width - 8.0,
        7.0,
        8.0,
        false,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "tenant-label",
        if is_arabic {
            "استلمنا من"
        } else {
            "RECEIVED FROM"
        },
        inset,
        77.0,
        content_width,
        5.0,
        8.0,
        true,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "tenant-name",
        "{{payment.tenant}}",
        inset,
        84.0,
        content_width,
        8.0,
        13.0,
        true,
        ink,
        "start",
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "amount-card",
        inset,
        97.0,
        content_width,
        27.0,
        "#F8F5EC",
        gold,
        0.8,
    ));
    push_text(
        children,
        "amount-label",
        if is_arabic {
            "إجمالي المبلغ المستلم"
        } else {
            "TOTAL RECEIVED"
        },
        inset + 4.0,
        100.0,
        content_width - 8.0,
        6.0,
        8.0,
        true,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "amount-value",
        "{{payment.amount}}",
        inset + 4.0,
        108.0,
        content_width - 8.0,
        12.0,
        19.0,
        true,
        navy,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "allocation-heading",
        if is_arabic {
            "توزيع الدفعة"
        } else {
            "PAYMENT ALLOCATION"
        },
        inset,
        130.0,
        content_width,
        6.0,
        9.0,
        true,
        navy,
        "start",
        locale,
        direction,
        font_family,
    );
    children.push(table_node(
        "payment-allocations",
        "payments",
        vec![
            json!({"header": if is_arabic { "العقد" } else { "Contract" }, "field": "contract_number", "width": 31, "align": "start"}),
            json!({"header": if is_arabic { "العقار" } else { "Property" }, "field": "property_name", "width": 40, "align": "start"}),
            json!({"header": if is_arabic { "الوحدة" } else { "Unit" }, "field": "unit_number", "width": 20, "align": "center"}),
            json!({"header": if is_arabic { "المبلغ" } else { "Amount" }, "field": "amount", "width": 35, "align": "end"}),
        ],
        inset,
        138.0,
        content_width,
        27.0,
        direction,
        navy,
        "#FFFFFF",
        "#DFE5ED",
    ));
    push_text(
        children,
        "payment-method",
        if is_arabic {
            "طريقة الدفع: {{payment.method}}"
        } else {
            "Method: {{payment.method}}"
        },
        inset,
        169.0,
        content_width,
        6.0,
        8.0,
        false,
        ink,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "payment-reference",
        if is_arabic {
            "المرجع: {{payment.reference}}"
        } else {
            "Reference: {{payment.reference}}"
        },
        inset,
        177.0,
        content_width,
        6.0,
        8.0,
        false,
        ink,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "payment-notes",
        if is_arabic {
            "ملاحظات: {{payment.notes}}"
        } else {
            "Notes: {{payment.notes}}"
        },
        inset,
        185.0,
        content_width,
        6.0,
        8.0,
        false,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "receipt-footer-rule",
        inset,
        194.0,
        content_width,
        0.5,
        "#DFE5ED",
        "#DFE5ED",
        0.0,
    ));
    push_text(
        children,
        "receipt-footer",
        if is_arabic {
            "شكراً لثقتكم · {{organization.name}}"
        } else {
            "Thank you for your trust · {{organization.name}}"
        },
        inset,
        198.0,
        content_width,
        6.0,
        7.0,
        false,
        muted,
        "center",
        locale,
        direction,
        font_family,
    );
}

fn add_lease_layout(
    children: &mut Vec<Value>,
    template: &DefaultContractTemplate,
    width: f64,
    inset: f64,
    navy: &str,
    ink: &str,
    muted: &str,
    soft: &str,
    locale: &str,
    direction: &str,
    font_family: &str,
) {
    let is_arabic = locale == "ar";
    let content_width = width - inset * 2.0;
    let title = match (template.lease_type, is_arabic) {
        ("commercial", true) => "عقد إيجار تجاري",
        ("short_term", true) => "اتفاق إيجار قصير الأجل",
        ("commercial", false) => "COMMERCIAL LEASE AGREEMENT",
        ("short_term", false) => "SHORT-TERM STAY AGREEMENT",
        (_, true) => "عقد إيجار سكني",
        (_, false) => "RESIDENTIAL LEASE AGREEMENT",
    };
    push_text(
        children,
        "lease-title",
        title,
        inset,
        45.0,
        content_width,
        13.0,
        22.0,
        true,
        navy,
        "center",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "lease-reference",
        if is_arabic {
            "رقم العقد: {{contract.number}}     ·     التاريخ: {{contract.date}}"
        } else {
            "Agreement no.: {{contract.number}}     ·     Date: {{contract.date}}"
        },
        inset,
        62.0,
        content_width,
        7.0,
        10.0,
        false,
        muted,
        "center",
        locale,
        direction,
        font_family,
    );
    section_heading(
        children,
        "parties-heading",
        if is_arabic {
            "أطراف العقد"
        } else {
            "THE PARTIES"
        },
        inset,
        78.0,
        content_width,
        navy,
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "parties-card",
        inset,
        87.0,
        content_width,
        32.0,
        soft,
        "#DFE5ED",
        0.4,
    ));
    for (index, line) in [
        if is_arabic {
            "المؤجر / ممثل العقار: {{organization.name}}"
        } else {
            "Landlord / representative: {{organization.name}}"
        },
        if is_arabic {
            "المستأجر: {{tenant.full_name}}"
        } else {
            "Tenant: {{tenant.full_name}}"
        },
        if is_arabic {
            "الهاتف: {{tenant.phone}}    ·    البريد الإلكتروني: {{tenant.email}}"
        } else {
            "Phone: {{tenant.phone}}    ·    Email: {{tenant.email}}"
        },
        if is_arabic {
            "العنوان: {{tenant.address}}"
        } else {
            "Address: {{tenant.address}}"
        },
    ]
    .iter()
    .enumerate()
    {
        push_text(
            children,
            &format!("party-{index}"),
            line,
            inset + 4.0,
            89.0 + index as f64 * 7.0,
            content_width - 8.0,
            6.0,
            9.0,
            index < 2,
            ink,
            "start",
            locale,
            direction,
            font_family,
        );
    }
    section_heading(
        children,
        "premises-heading",
        if is_arabic {
            "العين المؤجرة"
        } else {
            "THE PREMISES"
        },
        inset,
        126.0,
        content_width,
        navy,
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "premises-card",
        inset,
        135.0,
        content_width,
        34.0,
        "#FFFFFF",
        "#DFE5ED",
        0.5,
    ));
    for (index, line) in [
        if is_arabic { "العقار: {{property.name}} ({{property.code}})" } else { "Property: {{property.name}} ({{property.code}})" },
        if is_arabic { "العنوان: {{property.address}}، {{property.city}}" } else { "Address: {{property.address}}, {{property.city}}" },
        if is_arabic { "الوحدة: {{unit.number}} — {{unit.type}}    ·    الطابق: {{unit.floor}}" } else { "Unit: {{unit.number}} — {{unit.type}}    ·    Floor: {{unit.floor}}" },
        if is_arabic { "المساحة: {{unit.area}}    ·    غرف النوم: {{unit.bedrooms}}    ·    دورات المياه: {{unit.bathrooms}}" } else { "Area: {{unit.area}}    ·    Bedrooms: {{unit.bedrooms}}    ·    Bathrooms: {{unit.bathrooms}}" },
    ]
    .iter()
    .enumerate()
    {
        push_text(
            children,
            &format!("premises-{index}"),
            line,
            inset + 4.0,
            137.0 + index as f64 * 7.0,
            content_width - 8.0,
            6.0,
            8.5,
            false,
            ink,
            "start",
            locale,
            direction,
            font_family,
        );
    }
    section_heading(
        children,
        "terms-heading",
        if is_arabic {
            "المدة والقيمة"
        } else {
            "TERM AND AMOUNTS"
        },
        inset,
        176.0,
        content_width,
        navy,
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "terms-card",
        inset,
        185.0,
        content_width,
        31.0,
        soft,
        "#DFE5ED",
        0.4,
    ));
    let financial_lines = [
        if is_arabic {
            "مدة الإيجار: {{contract.start_date}} إلى {{contract.end_date}}"
        } else {
            "Lease term: {{contract.start_date}} to {{contract.end_date}}"
        },
        if is_arabic {
            "الأجرة المتفق عليها: {{unit.rent}} {{unit.currency}}"
        } else {
            "Agreed rent: {{unit.rent}} {{unit.currency}}"
        },
        if is_arabic {
            "مبلغ التأمين: {{contract.deposit}}"
        } else {
            "Security deposit: {{contract.deposit}}"
        },
        if template.lease_type == "commercial" {
            if is_arabic {
                "الغرض من الاستخدام: [يُستكمل بالاتفاق بين الطرفين]"
            } else {
                "Permitted use: [Complete as agreed by the parties]"
            }
        } else if template.lease_type == "short_term" {
            if is_arabic {
                "ترتيبات الإقامة: [تُستكمل أوقات الدخول والمغادرة]"
            } else {
                "Stay arrangements: [Complete check-in and check-out details]"
            }
        } else if is_arabic {
            "مواعيد السداد وطريقته: [تُستكمل بالاتفاق بين الطرفين]"
        } else {
            "Payment schedule and method: [Complete as agreed]"
        },
    ];
    for (index, line) in financial_lines.iter().enumerate() {
        push_text(
            children,
            &format!("financial-{index}"),
            line,
            inset + 4.0,
            187.0 + index as f64 * 7.0,
            content_width - 8.0,
            6.0,
            8.5,
            index == 1,
            ink,
            "start",
            locale,
            direction,
            font_family,
        );
    }
    section_heading(
        children,
        "special-terms-heading",
        if is_arabic {
            "بيانات وشروط خاصة"
        } else {
            "SPECIAL TERMS"
        },
        inset,
        223.0,
        content_width,
        navy,
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "special-terms-card",
        inset,
        232.0,
        content_width,
        37.0,
        "#FFFFFF",
        "#DFE5ED",
        0.5,
    ));
    push_text(
        children,
        "special-terms-placeholder",
        if is_arabic {
            "[تُستكمل بيانات التسليم، ومواعيد السداد، والخدمات والمرافق، وأي شروط متفق عليها. تُرفق التفاصيل الإضافية في ملحق موقع.]"
        } else {
            "[Complete handover details, payment dates, included services, and any agreed terms. Attach further details in a signed addendum.]"
        },
        inset + 4.0,
        236.0,
        content_width - 8.0,
        28.0,
        8.5,
        false,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "lease-footer-rule",
        inset,
        282.0,
        content_width,
        0.5,
        "#DFE5ED",
        "#DFE5ED",
        0.0,
    ));
    push_text(
        children,
        "lease-footer",
        if is_arabic {
            "مسودة قابلة للتخصيص · تُراجع قبل التوقيع"
        } else {
            "Editable draft · Review before signing"
        },
        inset,
        285.0,
        content_width,
        6.0,
        7.0,
        false,
        muted,
        "center",
        locale,
        direction,
        font_family,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_lease_legal_page(
    children: &mut Vec<Value>,
    template: &DefaultContractTemplate,
    width: f64,
    inset: f64,
    navy: &str,
    gold: &str,
    ink: &str,
    muted: &str,
    locale: &str,
    direction: &str,
    font_family: &str,
) {
    let is_arabic = locale == "ar";
    let content_width = width - inset * 2.0;
    push_text(
        children,
        "page-2-terms-title",
        if is_arabic {
            "الشروط والأحكام"
        } else {
            "TERMS AND CONDITIONS"
        },
        inset,
        45.0,
        content_width,
        12.0,
        19.0,
        true,
        navy,
        "center",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "page-2-terms-subtitle",
        if is_arabic {
            "اقرأ البنود وأكمل البيانات الخاصة قبل توقيع النسختين"
        } else {
            "Complete the agreed details before both parties sign"
        },
        inset,
        60.0,
        content_width,
        6.0,
        8.0,
        false,
        muted,
        "center",
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "page-2-title-rule",
        inset + content_width * 0.3,
        69.0,
        content_width * 0.4,
        0.8,
        gold,
        gold,
        0.0,
    ));

    let permitted_use = match (template.lease_type, is_arabic) {
        ("commercial", true) => {
            "يقتصر الاستعمال على النشاط المحدد كتابةً في الملحق. يستصدر المستأجر تصاريح نشاطه، دون إخلال بالتزامات المؤجر أو الأحكام الآمرة."
        }
        ("short_term", true) => {
            "تحدد أوقات الدخول والمغادرة وعدد الشاغلين والخدمات في الملحق. يحافظ المستأجر على الوحدة ومحتوياتها ولا يتيح الإقامة للغير خلافاً للاتفاق."
        }
        (_, true) => {
            "تستعمل الوحدة للسكن. لا يغير المستأجر الغرض أو يتنازل عن الانتفاع أو يؤجر من الباطن إلا بموافقة كتابية، مع مراعاة ما يجيزه القانون."
        }
        ("commercial", false) => {
            "Use is limited to the activity stated in a signed addendum. The Tenant obtains permits for its activity, without prejudice to the Landlord’s duties or mandatory law."
        }
        ("short_term", false) => {
            "Check-in and check-out times, permitted occupants, and included services must be stated in an addendum. The Tenant must care for the premises and may not provide occupancy contrary to this agreement."
        }
        (_, false) => {
            "The unit is for residential use. The Tenant may not change its use, assign it, or sublet without written consent, subject to rights that cannot be waived under applicable law."
        }
    };
    let clauses = if is_arabic {
        [
            (
                "المادة (1) — نطاق العقد",
                "يؤجر المؤجر للمستأجر الوحدة المبينة في الصفحة الأولى للغرض المحدد هنا. ويعد محضر التسليم والجرد والملحقات الموقعة جزءاً من العقد.",
            ),
            ("المادة (2) — الاستعمال", permitted_use),
            (
                "المادة (3) — الأجرة والتأمين",
                "تسدد الأجرة في المواعيد والطريقة المبينة. يوثق كل سداد بإيصال. يحدد الملحق معالجة التأمين عند انتهاء العقد، بما لا يخالف القانون، ولا تستحق رسوم أو غرامات غير متفق عليها كتابةً أو غير جائزة قانوناً.",
            ),
            (
                "المادة (4) — الصيانة والمرافق",
                "تحدد مسؤوليات الصيانة والمرافق كتابةً أو وفق القانون النافذ. يخطر كل طرف الآخر دون تأخير عن العطل أو الضرر الجوهري. لا يدخل المؤجر الوحدة إلا بموافقة المستأجر أو لحالة طارئة أو حيث يجيز القانون.",
            ),
            (
                "المادة (5) — التغييرات والتنازل",
                "لا يجري المستأجر تغييراً جوهرياً أو إضافة ثابتة، ولا يتنازل عن العقد أو يؤجر من الباطن، إلا بموافقة كتابية مسبقة وبما لا يخالف القانون. وتحدد كتابةً معالجة الإضافات عند انتهاء العقد.",
            ),
            (
                "المادة (6) — المدة والإنهاء",
                "ينتهي العقد بانقضاء مدته ما لم يتفق الطرفان كتابةً على التجديد أو يقضِ حكم آمر بخلاف ذلك. يكون الإشعار والإنهاء وتسليم الوحدة وفق الاتفاق والقانون النافذ.",
            ),
            (
                "المادة (7) — الإخلال والنزاع",
                "يخطر الطرف المتضرر الطرف الآخر كتابةً بالمخالفة، مع مراعاة مهلة المعالجة والإجراءات والحقوق المقررة قانوناً. يسعى الطرفان للتسوية الودية، وإلا يُعرض النزاع على الجهة القضائية اليمنية المختصة.",
            ),
            (
                "المادة (8) — أحكام ختامية",
                "لا يعدل العقد إلا بمحرر مكتوب موقع من الطرفين. يحرر من نسختين أصليتين، وتسلم لكل طرف نسخة. تعد الملحقات الموقعة جزءاً منه، ويحدد الطرفان النسخة المعتمدة إذا حرر بلغتين.",
            ),
        ]
    } else {
        [
            (
                "Article 1 — Scope",
                "The Landlord lets the unit identified on page one to the Tenant for the stated purpose. Signed handover records, inventories, and addenda form part of this agreement.",
            ),
            ("Article 2 — Permitted Use", permitted_use),
            (
                "Article 3 — Rent and Deposit",
                "Rent is payable on the stated dates and by the stated method; each payment must be receipted. The addendum must state deposit treatment at expiry, subject to law. No unwritten or unlawful fee or penalty is due.",
            ),
            (
                "Article 4 — Maintenance and Utilities",
                "Responsibilities for maintenance and utilities must be stated in writing or determined by applicable law. Each party must promptly report material damage. Entry requires consent, an emergency, or legal authorization.",
            ),
            (
                "Article 5 — Alterations and Assignment",
                "The Tenant may not make material alterations, install fixtures, assign this agreement, or sublet without prior written consent, subject to applicable law. Treatment of approved additions must be agreed in writing.",
            ),
            (
                "Article 6 — Term and Ending",
                "This agreement ends at expiry unless renewed in writing or a mandatory rule provides otherwise. Notice, termination, and handover follow the written agreement and applicable law.",
            ),
            (
                "Article 7 — Breach and Disputes",
                "The affected party gives written notice of a breach, subject to lawful procedures and any agreed remedy period. The parties will seek an amicable resolution; otherwise, the competent Yemeni court has jurisdiction under applicable rules.",
            ),
            (
                "Article 8 — General",
                "Amendments must be written and signed by both parties. This agreement is executed in two originals, one for each party; signed addenda form part of it. The parties identify the controlling version if bilingual copies are signed.",
            ),
        ]
    };

    for (index, (heading, body)) in clauses.iter().enumerate() {
        let y = 73.0 + index as f64 * 21.0;
        push_text(
            children,
            &format!("page-2-clause-heading-{index}"),
            heading,
            inset,
            y,
            content_width,
            6.0,
            9.0,
            true,
            navy,
            "start",
            locale,
            direction,
            font_family,
        );
        push_text(
            children,
            &format!("page-2-clause-body-{index}"),
            body,
            inset,
            y + 6.0,
            content_width,
            13.0,
            8.0,
            false,
            ink,
            "start",
            locale,
            direction,
            font_family,
        );
    }

    section_heading(
        children,
        "page-2-signatures-heading",
        if is_arabic {
            "الإقرار والتوقيع"
        } else {
            "ACKNOWLEDGMENT AND SIGNATURES"
        },
        inset,
        245.0,
        content_width,
        navy,
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "page-2-landlord-signature",
        if is_arabic {
            "المؤجر / الطرف الأول: ____________________  التوقيع: __________  التاريخ: ______"
        } else {
            "Landlord / First Party: __________________  Signature: _________  Date: ______"
        },
        inset,
        254.0,
        content_width,
        7.0,
        7.5,
        false,
        ink,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "page-2-tenant-signature",
        if is_arabic {
            "المستأجر / الطرف الثاني: __________________  التوقيع: __________  التاريخ: ______"
        } else {
            "Tenant / Second Party: ____________________  Signature: _________  Date: ______"
        },
        inset,
        264.0,
        content_width,
        7.0,
        7.5,
        false,
        ink,
        "start",
        locale,
        direction,
        font_family,
    );
    push_text(
        children,
        "page-2-witnesses",
        if is_arabic {
            "الشاهد الأول: ____________________________  الشاهد الثاني: ____________________________"
        } else {
            "Witness 1: ______________________________  Witness 2: ______________________________"
        },
        inset,
        274.0,
        content_width,
        6.0,
        7.0,
        false,
        muted,
        "start",
        locale,
        direction,
        font_family,
    );
    children.push(shape_node(
        "page-2-footer-rule",
        inset,
        283.0,
        content_width,
        0.5,
        "#DFE5ED",
        "#DFE5ED",
        0.0,
    ));
    push_text(
        children,
        "page-2-footer",
        if is_arabic {
            "مسودة عقد · تطبق القواعد الآمرة في الجمهورية اليمنية"
        } else {
            "Agreement draft · Mandatory laws of the Republic of Yemen apply"
        },
        inset,
        286.0,
        content_width,
        6.0,
        7.0,
        false,
        muted,
        "center",
        locale,
        direction,
        font_family,
    );
}

fn section_heading(
    children: &mut Vec<Value>,
    id: &str,
    content: &str,
    x: f64,
    y: f64,
    width: f64,
    color: &str,
    locale: &str,
    direction: &str,
    font_family: &str,
) {
    push_text(
        children,
        id,
        content,
        x,
        y,
        width,
        7.0,
        10.0,
        true,
        color,
        "start",
        locale,
        direction,
        font_family,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_text(
    children: &mut Vec<Value>,
    id: &str,
    content: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
    bold: bool,
    color: &str,
    align: &str,
    _locale: &str,
    direction: &str,
    font_family: &str,
) {
    children.push(text_node(
        id,
        content,
        x,
        y,
        width,
        height,
        font_size,
        bold,
        color,
        align,
        direction,
        font_family,
    ));
}

#[allow(clippy::too_many_arguments)]
fn text_node(
    id: &str,
    content: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
    bold: bool,
    color: &str,
    align: &str,
    direction: &str,
    font_family: &str,
) -> Value {
    json!({
        "type": "Text",
        "id": id,
        "name": id,
        "positionMode": "absolute",
        "props": {
            "content": content,
            "position": { "x": x, "y": y, "width": width, "height": height },
            "fontSize": font_size,
            "fontFamily": font_family,
            "fontWeight": if bold { "bold" } else { "normal" },
            "color": color,
            "textAlign": align,
            "direction": direction,
            "lineHeight": 1.35
        }
    })
}

fn shape_node(
    id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    fill: &str,
    stroke: &str,
    stroke_width: f64,
) -> Value {
    json!({
        "type": "Shape",
        "id": id,
        "name": id,
        "positionMode": "absolute",
        "props": {
            "shapeType": "rectangle",
            "position": { "x": x, "y": y, "width": width, "height": height },
            "fill": fill,
            "stroke": stroke,
            "strokeWidth": stroke_width,
            "borderRadius": 2.0
        }
    })
}

fn image_node(id: &str, src: &str, x: f64, y: f64, width: f64, height: f64) -> Value {
    json!({
        "type": "Image",
        "id": id,
        "name": id,
        "positionMode": "absolute",
        "props": {
            "src": src,
            "position": { "x": x, "y": y, "width": width, "height": height },
            "fit": "contain",
            "borderRadius": 2.0
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn table_node(
    id: &str,
    data_source: &str,
    columns: Vec<Value>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    direction: &str,
    header_background: &str,
    header_text: &str,
    border: &str,
) -> Value {
    json!({
        "type": "Table",
        "id": id,
        "name": id,
        "positionMode": "absolute",
        "props": {
            "position": { "x": x, "y": y, "width": width, "height": height },
            "columns": columns,
            "dataSource": data_source,
            "showHeader": true,
            "showBorder": true,
            "showGrid": true,
            "direction": direction,
            "rows": [],
            "style": {
                "headerBgColor": header_background,
                "headerTextColor": header_text,
                "alternateRowBgColor": "#F8FAFC",
                "borderColor": border,
                "borderWidth": 0.35,
                "headerFontSize": 7.0,
                "fontSize": 7.0,
                "rowHeight": 20.0
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_templates_are_positioned_docforge_documents_with_a5_receipts() {
        for template in &DEFAULT_CONTRACT_TEMPLATES {
            let placeholders: Value =
                serde_json::from_str(&default_template_placeholders(template))
                    .expect("parse default placeholders");
            let document: Value = serde_json::from_str(
                placeholders["docforge_document"]
                    .as_str()
                    .expect("read embedded DocForge document"),
            )
            .expect("parse DocForge document");
            let is_receipt = template.lease_type == "receipt";
            let expected_size = if is_receipt { "A5" } else { "A4" };
            let pages = document["pages"].as_array().expect("read DocForge pages");

            assert_eq!(document["version"], "2.0");
            assert_eq!(document["meta"]["pageSetup"]["size"], expected_size);
            assert_eq!(document["meta"]["pageSetup"]["orientation"], "portrait");
            assert_eq!(pages.len(), if is_receipt { 1 } else { 2 });
            for page in pages {
                let children = page["children"].as_array().expect("read page elements");
                assert!(children.iter().any(|node| node["id"] == "brand-band"
                    || node["id"] == "page-2-brand-band"));
                assert!(
                    children.iter().any(|node| {
                        node["id"].as_str().is_some_and(|id| id.contains("footer"))
                    })
                );
                assert!(children.iter().all(|node| {
                    node["props"]["position"]["x"].is_number()
                        && node["props"]["position"]["y"].is_number()
                        && node["props"]["position"]["width"].is_number()
                        && node["props"]["position"]["height"].is_number()
                }));
            }

            if is_receipt {
                let children = pages[0]["children"]
                    .as_array()
                    .expect("read receipt elements");
                assert!(children.iter().any(|node| node["type"] == "Table"));
                assert!(children.iter().any(|node| {
                    node["id"] == "amount-value" && node["props"]["content"] == "{{payment.amount}}"
                }));
            } else {
                assert!(
                    pages[1]["children"]
                        .as_array()
                        .expect("read contract terms")
                        .iter()
                        .any(|node| node["id"] == "page-2-clause-body-6")
                );
            }
        }
    }

    #[test]
    fn default_lease_content_has_two_pages_of_yemen_specific_terms() {
        for template in DEFAULT_CONTRACT_TEMPLATES
            .iter()
            .filter(|template| template.lease_type != "receipt")
        {
            let content = default_template_content(template);
            assert!(content.contains("[[PAGE_BREAK]]"));
            assert!(content.contains("المادة (7)") || content.contains("Article 7"));
            assert!(content.contains("الجمهورية اليمنية") || content.contains("Republic of Yemen"));
        }
    }
}
